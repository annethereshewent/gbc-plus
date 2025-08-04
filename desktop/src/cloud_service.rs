use std::{
    fs,
    path::PathBuf,
    time::{
        SystemTime,
        UNIX_EPOCH
    }
};

use dirs_next::data_dir;
use reqwest::{
    blocking::{
        Body,
        Client,
        Response
    },
    header::{
        HeaderMap,
        HeaderValue
    },
    Error,
    StatusCode
};
use serde::{Deserialize, Serialize};
use tiny_http::Server;

const CLIENT_ID: &str = "353451169812-8rpe4r0mt3rr2108nsgq8ctsukoo8fr7.apps.googleusercontent.com";

// according to google, since you can't keep secrets in desktop apps, keeping it in the source could should be ok.
// furthermore, google treats client secrets in native apps as extensions of the client ID, and not really a secret,
// and things like incremental login will not work with desktop apps, which is what the secret is used for on web
const CLIENT_SECRET: &str = "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";

const BASE_LOGIN_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";

const BASE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

// only two HTTP methods are used:
// one for updating and another for creating a new file
// save management is available on the web and iOS versions
// of the app
enum HttpMethod {
    Post,
    Patch
}

#[derive(Serialize, Deserialize)]
struct FileJson {
    name: String,
    // this needs to be in camel case for use with google API
    mimeType: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: isize,
    scope: String
}

#[derive(Serialize, Deserialize)]
struct DriveResponse {
    files: Vec<File>
}

#[derive(Serialize, Deserialize, Debug)]
struct File {
    id: String,
    name: String,
    parents: Vec<String>
}

pub struct CloudService {
    access_token: String,
    refresh_token: String,
    auth_code: String,
    pub logged_in: bool,
    client: Client,
    gbc_folder_id: String,
    pub game_name: String,
    expires_in: isize
}

impl CloudService {
    pub fn new(game_name: String) -> Self {
        let app_path = Self::get_access_token_path();

        let mut json = TokenResponse {
            access_token: "".to_string(),
            refresh_token: None,
            token_type: "".to_string(),
            expires_in: -1,
            scope: "".to_string()
        };

        if app_path.is_file() {
            json = serde_json::from_str(&fs::read_to_string(app_path).unwrap()).unwrap();
        }

        let expires_in = -1;

        Self {
            access_token: json.access_token.clone(),
            refresh_token: if json.refresh_token.is_some() { json.refresh_token.unwrap() } else { "".to_string() },
            auth_code: String::new(),
            logged_in: json.access_token != "",
            client: Client::new(),
            gbc_folder_id: String::new(),
            game_name,
            expires_in
        }
    }

    fn get(&self, url: &str) -> Response {
        self.client
            .get(url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .unwrap()
    }

    fn post(
        &self,
        url: &str,
        body_str: Option<String>,
        bytes: Option<Vec<u8>>,
        headers: Option<HeaderMap<HeaderValue>>
    ) -> Response {
        self.request(url, HttpMethod::Post, body_str, bytes, headers)
    }

    fn patch(
        &self,
        url: &str,
        body_str: Option<String>,
        bytes: Option<Vec<u8>>,
        headers: Option<HeaderMap<HeaderValue>>
    ) -> Response {
        self.request(url, HttpMethod::Patch, body_str, bytes, headers)
    }

    fn request(
        &self,
        url: &str,
        method: HttpMethod,
        body_str: Option<String>,
        bytes: Option<Vec<u8>>,
        headers: Option<HeaderMap<HeaderValue>>
    ) -> Response {
        let mut    builder = match method {
            HttpMethod::Patch => self.client.patch(url),
            HttpMethod::Post => self.client.post(url)
        };

        let body = if let Some(body_str) = body_str {
            Some(Body::from(body_str))
        } else if let Some(bytes) = bytes {
            Some(Body::from(bytes))
        } else {
            None
        };

        if let Some(body) = body {
            builder = builder.body(body);
        }

        builder = builder.header("Authorization", format!("Bearer {}", self.access_token));

        if let Some(headers) = headers {
            builder = builder.headers(headers);
        }

        builder.send().unwrap()
    }

    fn get_access_token_path() -> PathBuf {
        let mut app_support_dir = data_dir().unwrap();

        app_support_dir.push("GBC+");

        app_support_dir.push("access_token.json");

        app_support_dir
}

    fn refresh_login(&mut self) {
        let mut body_params: Vec<[&str; 2]> = Vec::new();

        body_params.push(["client_id", CLIENT_ID]);
        body_params.push(["client_secret", CLIENT_SECRET]);
        body_params.push(["grant_type", "refresh_token"]);
        body_params.push(["refresh_token", &self.refresh_token]);

        let params = Self::generate_params_string(body_params);

        let token_response = self.client
            .post(BASE_TOKEN_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from(params))
            .send()
            .unwrap();

        if token_response.status() == StatusCode::OK {
            let json: Result<TokenResponse, Error> = token_response.json();

            if json.is_ok() {
                let mut json = json.unwrap();

                self.access_token = json.clone().access_token;

                let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("an error occurred")
                    .as_millis();

                self.expires_in = current_time as isize + json.expires_in as isize * 1000;

                json.expires_in = self.expires_in;
                json.refresh_token = Some(self.refresh_token.clone());

                let access_token_path = Self::get_access_token_path();

                fs::write(access_token_path, serde_json::to_string(&json).unwrap()).unwrap();
            } else {
                let error = json.err().unwrap();

                self.logout();

                println!("error refreshing login: {:?}", error);
            }
        } else {
            self.logout();

            println!("error refreshing login: {}", token_response.text().unwrap());
        }
    }

    pub fn check_for_gbc_folder(&mut self) {
        self.refresh_token_if_needed();
        let mut query_params: Vec<[&str; 2]> = Vec::new();

        query_params.push(["q", "mimeType = \"application/vnd.google-apps.folder\" and name=\"gbc-saves\""]);
        query_params.push(["fields", "files/id,files/parents,files/name"]);

        let query_string = Self::generate_params_string(query_params);

        let url = format!("https://www.googleapis.com/drive/v3/files?{query_string}");

        let response = self.get(&url);

        if response.status() == StatusCode::OK {
            self.process_folder_response(response);
        } else {
            println!("An error occurred while using Google API: Response code: {}", response.status());
        }
    }

    fn process_folder_response(&mut self, response: Response) {
        self.refresh_token_if_needed();

        let json: DriveResponse = response.json().unwrap();

        if let Some(folder) = json.files.get(0) {
            self.gbc_folder_id = folder.id.clone();
        } else {
            // create the gbc folder

            let url = "https://www.googleapis.com/drive/v3/files?uploadType=media";

            let folder_json = FileJson {
                name: "gbc-saves".to_string(),
                mimeType: "application/vnd.google-apps.folder".to_string()
            };

            let json_str = serde_json::to_string(&folder_json).unwrap();

            let mut headers = HeaderMap::new();

            headers.append("Content-Type", HeaderValue::from_str("application/vnd.google-apps.folder").unwrap());

            let response = self.post(
                url,
                Some(json_str.clone()),
                None,
                Some(headers.clone())
            );

            if response.status() == StatusCode::OK {
                let json: DriveResponse = response.json().unwrap();

                if let Some(folder) = json.files.get(0) {
                    self.gbc_folder_id = folder.id.clone();
                } else {
                    println!("Could not create GBC folder");
                }
            } else {
                println!("Could not create GBC folder");
            }
        }
    }

    fn refresh_token_if_needed(&mut self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("an error occurred")
            .as_millis();

        if current_time as isize >= self.expires_in {
            self.refresh_login();
        }
    }

    // TODO: Fix this really long unfortunate method
    pub fn upload_file(&mut self, bytes: &[u8], rtc_name: Option<String>) {
        let game_name = if let Some(rtc_name) = rtc_name {
            rtc_name
        } else if self.game_name != "" {
            self.game_name.clone()
        } else {
            return
        };

        println!("uploading file!");

        self.refresh_token_if_needed();

        if self.gbc_folder_id == "" {
            self.check_for_gbc_folder();
        }

        let json = self.get_file_info(game_name.clone());

        let mut headers = HeaderMap::new();

        headers.append("Content-Type", HeaderValue::from_str("application/octet-stream").unwrap());
        headers.append("Content-Length", HeaderValue::from_str(&format!("{}", bytes.len())).unwrap());

        if let Some(file) = json.files.get(0) {
            let url = format!("https://www.googleapis.com/upload/drive/v3/files/{}?uploadType=media", file.id);

            let response = self.patch(
                &url,
                None,
                Some(bytes.to_vec()),
                Some(headers.clone())
            );

            if response.status() != StatusCode::OK {
                println!("Warning: Couldn't upload save to cloud! status code: {}", response.status());
            }

            return;
        }

        let url = "https://www.googleapis.com/upload/drive/v3/files?uploadType=media&fields=id,name,parents";

        let response = self.post(
            &url,
            None,
            Some(bytes.to_vec()),
            Some(headers.clone())
        );

        if response.status() == StatusCode::OK {
            // move and rename file
            self.rename_save(response, game_name);
        } else {
            println!("Warning: Couldn't upload save to cloud! status code: {}", response.status());
        }
    }

    fn rename_save(&mut self, response: Response, file_name: String) {
        self.refresh_token_if_needed();

        let file: File = response.json().unwrap();
        let mut query_params: Vec<[&str; 2]> = Vec::new();

        query_params.push(["uploadType", "media"]);
        query_params.push(["addParents", &self.gbc_folder_id]);

        let query_string = Self::generate_params_string(query_params);

        let url = format!("https://www.googleapis.com/drive/v3/files/{}?{}", file.id, query_string);

        let json = FileJson {
            name: file_name,
            mimeType: "application/octet-stream".to_string()
        };

        let json_str = serde_json::to_string(&json).unwrap();

        let response = self.patch(
            &url,
            Some(json_str.clone()),
            None,
            None
        );

        if response.status() != StatusCode::OK {
            println!("Warning: Couldn't rename save! status code = {}", response.status());
        }
    }

    pub fn get_file(&mut self, rtc_name: Option<String>) -> Vec<u8> {
        let file_name = if let Some(rtc_name) = rtc_name {
            rtc_name
        } else if self.game_name != "" {
            self.game_name.clone()
        } else {
            return Vec::new()
        };

        self.refresh_token_if_needed();

        self.check_for_gbc_folder();

        let json = self.get_file_info(file_name);

        if let Some(file) = json.files.get(0) {
            let url = format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", file.id);

            // time for some repetition! woo!
            let response = self.get(&url);

            if response.status() == StatusCode::OK {
                return response.bytes().unwrap().to_vec();
            }
        }

        Vec::new()
    }

    fn get_file_info(&mut self, file_name: String) -> DriveResponse {
        self.refresh_token_if_needed();
        let mut query_params: Vec<[&str; 2]> = Vec::new();

        let query = &format!("name = \"{}\" and parents in \"{}\"", file_name, self.gbc_folder_id);

        // rust complaining here if i just pass &String::new() to the encode method below,
        // so i have to initialize this variable here
        let mut _useless = String::new();

        query_params.push(["q", url_escape::encode_component_to_string(query, &mut _useless)]);
        query_params.push(["fields", "files/id,files/parents,files/name"]);

        let query_string = Self::generate_params_string(query_params);

        let url = format!("https://www.googleapis.com/drive/v3/files?{query_string}");

        let response = self.get(&url);

        if response.status() == StatusCode::OK {
            return response.json::<DriveResponse>().unwrap();
        }

        panic!("{:?}", response.text());
    }

    pub fn generate_params_string(params: Vec<[&str; 2]>) -> String {
        let param_arr: Vec<String> = params
            .iter()
            .map(|param| format!("{}={}", param[0], param[1]))
            .collect();

        // now after doing the collect we can finally actually create the query string
        let string = param_arr.join("&");

        string
    }

    pub fn login(&mut self) {
        let mut query_params: Vec<[&str; 2]> = Vec::new();

        query_params.push(["response_type", "code"]);
        query_params.push(["client_id", CLIENT_ID]);
        query_params.push(["redirect_uri", "http://localhost:8090"]);
        query_params.push(["scope", "https://www.googleapis.com/auth/drive.file https://www.googleapis.com/auth/userinfo.email"]);

        let query_string = Self::generate_params_string(query_params);

        open::that(format!("{BASE_LOGIN_URL}?{query_string}")).unwrap();

        let server = Server::http("127.0.0.1:8090").unwrap();

        'outer: for request in server.incoming_requests() {
            if let Some(query) = request.url().split_once("?") {
                let params = query.1.split("&");

                for param in params.into_iter() {
                    if let Some((key, value)) = param.split_once("=") {
                        if key == "code" {
                            self.auth_code = value.to_string();

                            request.respond(tiny_http::Response::from_string("Successfully logged in to Google! This tab can now be closed.")).unwrap();
                            break 'outer;
                        }
                    }
                }
            }
        }

        // make a request to google to get an auth token and refresh token
        let mut body_params: Vec<[&str; 2]> = Vec::new();

        body_params.push(["code", &self.auth_code]);
        body_params.push(["client_id", CLIENT_ID]);
        body_params.push(["client_secret", CLIENT_SECRET]);
        body_params.push(["redirect_uri", "http://localhost:8090"]);
        body_params.push(["grant_type", "authorization_code"]);

        let params = Self::generate_params_string(body_params);

        let response = self.client.post(BASE_TOKEN_URL)
            .body(
                Body::from(format!("{params}"))
            )
            .header("Content-Type", "application/x-www-form-urlencoded")
            .send()
            .unwrap();


        if response.status() == StatusCode::OK {
            let mut json: TokenResponse = response.json().unwrap();

            let current_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("an error occurred")
                    .as_millis();

            self.access_token = json.clone().access_token;
            self.expires_in = json.clone().expires_in * 1000 + current_time as isize;
            self.refresh_token = json.clone().refresh_token.unwrap();

            json.expires_in = self.expires_in;

            self.logged_in = true;

            let access_token_path = Self::get_access_token_path();

            // store these in files for use later
            fs::write(access_token_path, serde_json::to_string(&json.clone()).unwrap()).unwrap();
        }
    }

    pub fn logout(&mut self) {
        let access_token_path = Self::get_access_token_path();

        fs::remove_file(access_token_path).unwrap();

        self.access_token = String::new();
        self.refresh_token = String::new();
        self.logged_in = false;
    }
}