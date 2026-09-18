import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CraneIcon` instead.
 *
 * @example
 * ```svelte
 * <Crane color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Crane: Component<IconComponentProps, {}, "">;
type Crane = ReturnType<typeof Crane>;
export default Crane;
