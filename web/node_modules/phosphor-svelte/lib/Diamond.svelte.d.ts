import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `DiamondIcon` instead.
 *
 * @example
 * ```svelte
 * <Diamond color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Diamond: Component<IconComponentProps, {}, "">;
type Diamond = ReturnType<typeof Diamond>;
export default Diamond;
