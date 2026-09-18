import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MusicNoteSimpleIcon` instead.
 *
 * @example
 * ```svelte
 * <MusicNoteSimple color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MusicNoteSimple: Component<IconComponentProps, {}, "">;
type MusicNoteSimple = ReturnType<typeof MusicNoteSimple>;
export default MusicNoteSimple;
