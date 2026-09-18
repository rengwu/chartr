import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NoteIcon` instead.
 *
 * @example
 * ```svelte
 * <Note color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Note: Component<IconComponentProps, {}, "">;
type Note = ReturnType<typeof Note>;
export default Note;
