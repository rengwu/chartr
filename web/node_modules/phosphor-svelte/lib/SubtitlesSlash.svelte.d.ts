import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SubtitlesSlashIcon` instead.
 *
 * @example
 * ```svelte
 * <SubtitlesSlash color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SubtitlesSlash: Component<IconComponentProps, {}, "">;
type SubtitlesSlash = ReturnType<typeof SubtitlesSlash>;
export default SubtitlesSlash;
