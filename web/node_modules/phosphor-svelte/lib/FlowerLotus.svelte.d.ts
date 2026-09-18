import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FlowerLotusIcon` instead.
 *
 * @example
 * ```svelte
 * <FlowerLotus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FlowerLotus: Component<IconComponentProps, {}, "">;
type FlowerLotus = ReturnType<typeof FlowerLotus>;
export default FlowerLotus;
