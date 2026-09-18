import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `FileIniIcon` instead.
 *
 * @example
 * ```svelte
 * <FileIni color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const FileIni: Component<IconComponentProps, {}, "">;
type FileIni = ReturnType<typeof FileIni>;
export default FileIni;
