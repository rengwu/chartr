import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BriefcaseIcon` instead.
 *
 * @example
 * ```svelte
 * <Briefcase color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Briefcase: Component<IconComponentProps, {}, "">;
type Briefcase = ReturnType<typeof Briefcase>;
export default Briefcase;
