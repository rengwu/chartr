import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `ArrowLeftIcon` instead.
 *
 * @example
 * ```svelte
 * <ArrowLeft color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ArrowLeft: Component<IconComponentProps, {}, "">;
type ArrowLeft = ReturnType<typeof ArrowLeft>;
export default ArrowLeft;
