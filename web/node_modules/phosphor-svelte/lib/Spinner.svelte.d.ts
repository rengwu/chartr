import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SpinnerIcon` instead.
 *
 * @example
 * ```svelte
 * <Spinner color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Spinner: Component<IconComponentProps, {}, "">;
type Spinner = ReturnType<typeof Spinner>;
export default Spinner;
