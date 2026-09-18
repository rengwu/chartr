import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MusicNotesIcon` instead.
 *
 * @example
 * ```svelte
 * <MusicNotes color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MusicNotes: Component<IconComponentProps, {}, "">;
type MusicNotes = ReturnType<typeof MusicNotes>;
export default MusicNotes;
