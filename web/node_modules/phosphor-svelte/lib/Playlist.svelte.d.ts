import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlaylistIcon` instead.
 *
 * @example
 * ```svelte
 * <Playlist color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Playlist: Component<IconComponentProps, {}, "">;
type Playlist = ReturnType<typeof Playlist>;
export default Playlist;
