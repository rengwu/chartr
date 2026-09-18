import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `LightningAIcon` instead.
 *
 * @example
 * ```svelte
 * <LightningA color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const LightningA: Component<IconComponentProps, {}, "">;
type LightningA = ReturnType<typeof LightningA>;
export default LightningA;
