import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <EscalatorUpIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const EscalatorUpIcon: Component<IconComponentProps, {}, "">;
type EscalatorUpIcon = ReturnType<typeof EscalatorUpIcon>;
export default EscalatorUpIcon;
