inputs: {
  config,
  lib,
  pkgs,
  ...
}: with lib; let
  cfg = config.programs.comb;

  defaultCombPackage = inputs.self.packages.${pkgs.stdenv.hostPlatform.system}.default;

  writeUdevRule =
    pkgs.writeTextFile {
      name = "comb-uinput-udev-rule";
      text = ''
        SUBSYSTEM=="misc", KERNEL=="uinput", MODE="0660", GROUP="${cfg.uinputGroup}"
      '';
      destination = "/lib/udev/rules.d/85-comb-uinput.rules";
    };
in {
  options.programs.comb = {
    enable = mkEnableOption ''
      CoMB (Corroded Macro Bindings). A program to map gamepad and other
      evdev devices to events emitted by a virtual Uinput device
    '';

    package = mkOption {
      type = types.nullOr types.package;
      default = defaultCombPackage;
      defaultText = literalExpression "<CoMB flake>.packages.<system>.default";
      example = literalExpession "<CoMB flake>.packages.<system>.default.override { }";
      description = ''
        CoMB pckage to use.
      '';
    };

    udevRules = mkOption {
      type = types.bool;
      default = true;
      defaultText = literalExpression "true";
      example = literalExpression "false";
      description = ''
        Whether to add the recommended Udev rules.
      '';
    };

    uinputGroup = mkOption {
      type = types.str;
      default = "comb_uinput";
      defaultText = literalExpression "comb_uinput";
      example = literalExpression "uinput";
      description = ''
        The group to give access to /dev/uinput to.
      '';
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.udev.packages = mkIf cfg.udevRules [ writeUdevRule ];
    users.groups.${cfg.uinputGroup} = mkIf cfg.udevRules { };
  };
}
