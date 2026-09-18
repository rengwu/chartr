import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BookOpenIcon` instead.
 *
 * @example
 * ```svelte
 * <BookOpen color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BookOpen: Component<IconComponentProps, {}, "">;
type BookOpen = ReturnType<typeof BookOpen>;
export default BookOpen;
