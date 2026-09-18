import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PlusSquareIcon` instead.
 *
 * @example
 * ```svelte
 * <PlusSquare color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PlusSquare: Component<IconComponentProps, {}, "">;
type PlusSquare = ReturnType<typeof PlusSquare>;
export default PlusSquare;
