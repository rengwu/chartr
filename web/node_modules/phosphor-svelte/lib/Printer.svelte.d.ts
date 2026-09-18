import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PrinterIcon` instead.
 *
 * @example
 * ```svelte
 * <Printer color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Printer: Component<IconComponentProps, {}, "">;
type Printer = ReturnType<typeof Printer>;
export default Printer;
