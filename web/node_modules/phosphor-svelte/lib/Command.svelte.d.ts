import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CommandIcon` instead.
 *
 * @example
 * ```svelte
 * <Command color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Command: Component<IconComponentProps, {}, "">;
type Command = ReturnType<typeof Command>;
export default Command;
