import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FirstAidKitIcon` instead.
 *
 * @example
 * ```svelte
 * <FirstAidKit color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FirstAidKit: Component<IconComponentProps, {}, "">;
type FirstAidKit = ReturnType<typeof FirstAidKit>;
export default FirstAidKit;
