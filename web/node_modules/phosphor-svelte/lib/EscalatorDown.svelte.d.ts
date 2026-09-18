import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EscalatorDownIcon` instead.
 *
 * @example
 * ```svelte
 * <EscalatorDown color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const EscalatorDown: Component<IconComponentProps, {}, "">;
type EscalatorDown = ReturnType<typeof EscalatorDown>;
export default EscalatorDown;
