import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ShovelIcon` instead.
 *
 * @example
 * ```svelte
 * <Shovel color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Shovel: Component<IconComponentProps, {}, "">;
type Shovel = ReturnType<typeof Shovel>;
export default Shovel;
