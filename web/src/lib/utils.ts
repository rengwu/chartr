import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'

// The shadcn-svelte class merge helper: clsx flattens conditional class lists,
// tailwind-merge resolves conflicting Tailwind utilities so the last one wins.
// Vendored shadcn-svelte primitives import this from `$lib/utils`.
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs))
}
