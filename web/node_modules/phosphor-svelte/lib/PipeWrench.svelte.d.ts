import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `PipeWrenchIcon` instead.
 *
 * @example
 * ```svelte
 * <PipeWrench color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const PipeWrench: Component<IconComponentProps, {}, "">;
type PipeWrench = ReturnType<typeof PipeWrench>;
export default PipeWrench;
