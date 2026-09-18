import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CassetteTapeIcon` instead.
 *
 * @example
 * ```svelte
 * <CassetteTape color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CassetteTape: Component<IconComponentProps, {}, "">;
type CassetteTape = ReturnType<typeof CassetteTape>;
export default CassetteTape;
