import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `MinusCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <MinusCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const MinusCircle: Component<IconComponentProps, {}, "">;
type MinusCircle = ReturnType<typeof MinusCircle>;
export default MinusCircle;
