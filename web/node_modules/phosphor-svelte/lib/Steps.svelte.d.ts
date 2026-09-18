import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StepsIcon` instead.
 *
 * @example
 * ```svelte
 * <Steps color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Steps: Component<IconComponentProps, {}, "">;
type Steps = ReturnType<typeof Steps>;
export default Steps;
