import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SyringeIcon` instead.
 *
 * @example
 * ```svelte
 * <Syringe color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Syringe: Component<IconComponentProps, {}, "">;
type Syringe = ReturnType<typeof Syringe>;
export default Syringe;
