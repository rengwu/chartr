import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `HouseLineIcon` instead.
 *
 * @example
 * ```svelte
 * <HouseLine color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const HouseLine: Component<IconComponentProps, {}, "">;
type HouseLine = ReturnType<typeof HouseLine>;
export default HouseLine;
