export function reactive<T>(initial: T) {
  const listeners: Set<() => void> = new Set()
  let value = initial

  return {
    get value() {
      return value
    },
    set value(v: T) {
      value = v
      listeners.forEach(cb => cb())
    },
    subscribe(cb: () => void) {
      listeners.add(cb)
      return () => listeners.delete(cb)
    }
  }
}