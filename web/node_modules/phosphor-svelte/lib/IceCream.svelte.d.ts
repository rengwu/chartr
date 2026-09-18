import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `IceCreamIcon` instead.
 *
 * @example
 * ```svelte
 * <IceCream color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const IceCream: Component<IconComponentProps, {}, "">;
type IceCream = ReturnType<typeof IceCream>;
export default IceCream;
