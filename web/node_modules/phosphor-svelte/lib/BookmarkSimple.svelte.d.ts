import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BookmarkSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <BookmarkSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BookmarkSimple: Component<IconComponentProps, {}, "">;
type BookmarkSimple = ReturnType<typeof BookmarkSimple>;
export default BookmarkSimple;
