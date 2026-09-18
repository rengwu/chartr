import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CircleIcon` instead.
 *
 * @example
 * ```svelte
 * <Circle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Circle: Component<IconComponentProps, {}, "">;
type Circle = ReturnType<typeof Circle>;
export default Circle;
