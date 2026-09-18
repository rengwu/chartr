import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PrescriptionIcon` instead.
 *
 * @example
 * ```svelte
 * <Prescription color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Prescription: Component<IconComponentProps, {}, "">;
type Prescription = ReturnType<typeof Prescription>;
export default Prescription;
