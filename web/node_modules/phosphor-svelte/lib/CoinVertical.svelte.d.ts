import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CoinVerticalIcon` instead.
 *
 * @example
 * ```svelte
 * <CoinVertical color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CoinVertical: Component<IconComponentProps, {}, "">;
type CoinVertical = ReturnType<typeof CoinVertical>;
export default CoinVertical;
