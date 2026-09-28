declare const __badge: unique symbol

export type Badge<T, S extends string> = T & { [__badge]: S }
