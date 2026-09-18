import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BriefcaseMetalIcon` instead.
 *
 * @example
 * ```svelte
 * <BriefcaseMetal color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const BriefcaseMetal: Component<IconComponentProps, {}, "">;
type BriefcaseMetal = ReturnType<typeof BriefcaseMetal>;
export default BriefcaseMetal;
