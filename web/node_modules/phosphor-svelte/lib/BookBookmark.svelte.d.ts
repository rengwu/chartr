import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BookBookmarkIcon` instead.
 *
 * @example
 * ```svelte
 * <BookBookmark color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BookBookmark: Component<IconComponentProps, {}, "">;
type BookBookmark = ReturnType<typeof BookBookmark>;
export default BookBookmark;
