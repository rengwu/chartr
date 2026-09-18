import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @example
 * ```svelte
 * <ApproximateEqualsIcon color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const ApproximateEqualsIcon: Component<IconComponentProps, {}, "">;
type ApproximateEqualsIcon = ReturnType<typeof ApproximateEqualsIcon>;
export default ApproximateEqualsIcon;
