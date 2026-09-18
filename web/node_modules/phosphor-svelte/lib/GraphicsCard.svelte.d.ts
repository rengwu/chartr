import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `GraphicsCardIcon` instead.
 *
 * @example
 * ```svelte
 * <GraphicsCard color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const GraphicsCard: Component<IconComponentProps, {}, "">;
type GraphicsCard = ReturnType<typeof GraphicsCard>;
export default GraphicsCard;
