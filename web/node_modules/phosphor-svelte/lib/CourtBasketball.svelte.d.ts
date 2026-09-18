import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CourtBasketballIcon` instead.
 *
 * @example
 * ```svelte
 * <CourtBasketball color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CourtBasketball: Component<IconComponentProps, {}, "">;
type CourtBasketball = ReturnType<typeof CourtBasketball>;
export default CourtBasketball;
