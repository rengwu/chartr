import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SimCardIcon` instead.
 *
 * @example
 * ```svelte
 * <SimCard color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SimCard: Component<IconComponentProps, {}, "">;
type SimCard = ReturnType<typeof SimCard>;
export default SimCard;
