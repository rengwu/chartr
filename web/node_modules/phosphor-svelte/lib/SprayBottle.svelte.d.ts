import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SprayBottleIcon` instead.
 *
 * @example
 * ```svelte
 * <SprayBottle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SprayBottle: Component<IconComponentProps, {}, "">;
type SprayBottle = ReturnType<typeof SprayBottle>;
export default SprayBottle;
