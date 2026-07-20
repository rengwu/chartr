import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

// The shadcn-svelte class merge helper: clsx flattens conditional class lists,
// tailwind-merge resolves conflicting Tailwind utilities so the last one wins.
// Vendored shadcn-svelte primitives import this from `$lib/utils`.
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs))
}

// The shadcn-svelte prop-shape helpers vendored primitives are typed against:
// a bindable `ref` on top of an element's own attributes, and the two ways a
// Bits UI primitive's children can be stripped out when a wrapper renders its
// own (`child` — the render-prop escape hatch — and/or `children`).
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null }

export type WithoutChild<T> = T extends { child?: unknown } ? Omit<T, 'child'> : T
export type WithoutChildren<T> = T extends { children?: unknown } ? Omit<T, 'children'> : T
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>

