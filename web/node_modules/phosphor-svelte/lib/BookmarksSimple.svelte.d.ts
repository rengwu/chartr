import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BookmarksSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <BookmarksSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BookmarksSimple: Component<IconComponentProps, {}, "">;
type BookmarksSimple = ReturnType<typeof BookmarksSimple>;
export default BookmarksSimple;
