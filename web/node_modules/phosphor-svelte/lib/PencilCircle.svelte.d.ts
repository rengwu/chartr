import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PencilCircleIcon` instead.
 *
 * @example
 * ```svelte
 * <PencilCircle color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PencilCircle: Component<IconComponentProps, {}, "">;
type PencilCircle = ReturnType<typeof PencilCircle>;
export default PencilCircle;
