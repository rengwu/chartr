import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DotsThreeIcon` instead.
 *
 * @example
 * ```svelte
 * <DotsThree color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const DotsThree: Component<IconComponentProps, {}, "">;
type DotsThree = ReturnType<typeof DotsThree>;
export default DotsThree;
