import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `EscalatorUpIcon` instead.
 *
 * @example
 * ```svelte
 * <EscalatorUp color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const EscalatorUp: Component<IconComponentProps, {}, "">;
type EscalatorUp = ReturnType<typeof EscalatorUp>;
export default EscalatorUp;
