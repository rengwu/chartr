import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SealWarningIcon` instead.
 *
 * @example
 * ```svelte
 * <SealWarning color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SealWarning: Component<IconComponentProps, {}, "">;
type SealWarning = ReturnType<typeof SealWarning>;
export default SealWarning;
