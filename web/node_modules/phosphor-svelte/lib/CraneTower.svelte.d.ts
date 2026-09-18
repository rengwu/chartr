import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CraneTowerIcon` instead.
 *
 * @example
 * ```svelte
 * <CraneTower color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CraneTower: Component<IconComponentProps, {}, "">;
type CraneTower = ReturnType<typeof CraneTower>;
export default CraneTower;
