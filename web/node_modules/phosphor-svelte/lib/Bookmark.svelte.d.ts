import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BookmarkIcon` instead.
 *
 * @example
 * ```svelte
 * <Bookmark color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bookmark: Component<IconComponentProps, {}, "">;
type Bookmark = ReturnType<typeof Bookmark>;
export default Bookmark;
