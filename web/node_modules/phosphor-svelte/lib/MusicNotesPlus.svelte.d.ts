import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MusicNotesPlusIcon` instead.
 *
 * @example
 * ```svelte
 * <MusicNotesPlus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MusicNotesPlus: Component<IconComponentProps, {}, "">;
type MusicNotesPlus = ReturnType<typeof MusicNotesPlus>;
export default MusicNotesPlus;
