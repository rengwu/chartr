import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BookmarksIcon` instead.
 *
 * @example
 * ```svelte
 * <Bookmarks color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Bookmarks: Component<IconComponentProps, {}, "">;
type Bookmarks = ReturnType<typeof Bookmarks>;
export default Bookmarks;
