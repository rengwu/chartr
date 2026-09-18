import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NoteBlankIcon` instead.
 *
 * @example
 * ```svelte
 * <NoteBlank color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const NoteBlank: Component<IconComponentProps, {}, "">;
type NoteBlank = ReturnType<typeof NoteBlank>;
export default NoteBlank;
