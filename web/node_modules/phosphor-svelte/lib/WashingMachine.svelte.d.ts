import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `WashingMachineIcon` instead.
 *
 * @example
 * ```svelte
 * <WashingMachine color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const WashingMachine: Component<IconComponentProps, {}, "">;
type WashingMachine = ReturnType<typeof WashingMachine>;
export default WashingMachine;
