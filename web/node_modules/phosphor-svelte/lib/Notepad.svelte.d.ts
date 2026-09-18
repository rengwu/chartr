import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NotepadIcon` instead.
 *
 * @example
 * ```svelte
 * <Notepad color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Notepad: Component<IconComponentProps, {}, "">;
type Notepad = ReturnType<typeof Notepad>;
export default Notepad;
