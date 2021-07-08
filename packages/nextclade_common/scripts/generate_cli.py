#!/usr/bin/env python3

"""

Generates a .cpp and a .h file running CLI parser, according to a JSON file describing CLI options

"""

import argparse
import json
import os

THIS_DIR = os.path.dirname(os.path.realpath(__file__))
PROJECT_ROOT_DIR = os.path.realpath(os.path.join(THIS_DIR, "..", "..", ".."))
THIS_SCRIPT_REL = os.path.relpath(__file__, PROJECT_ROOT_DIR)

def parse_args():
    parser = argparse.ArgumentParser(
        description="Generates C++ code of the command-line interface described by the JSON file")
    parser.add_argument(
        "--input_json",
        required=True,
        help="Path to the input JSON file describing the CLI.")
    parser.add_argument(
        "--output_cpp",
        required=True,
        help="Path where to put the generated .cpp file."
    )
    parser.add_argument(
        "--output_h",
        required=True,
        help="Path where to put the generated .h file."
    )
    return parser.parse_args()


##########

CPP_HEADER = """



#include "cli.h"

#include <CLI/CLI.hpp>
#include <string>

namespace Nextclade {

CliParams parseCommandLine(int argc, char *argv[], const std::string& appDescription) {

CLI::App app(appDescription);

CliParams cliParams;

"""

##########

CPP_FOOTER = """

app.parse(argc, argv);

return cliParams;
}

}// namespace Nextclade

"""

##########

H_HEADER = """

#include <string>

namespace Nextclade {

struct CliParams {

"""

##########

H_FOOTER = """
};

CliParams parseCommandLine(int argc, char *argv[], const std::string& appDescription);

}// namespace Nextclade

"""


def generate_code(desc):
    cpp = ""
    h = ""

    for root_option in desc["root_options"]:
        flags = root_option["flags"]
        desc = root_option["desc"]
        cpp_name = root_option["cppName"]
        cpp_type = root_option["cppType"]
        is_optional = root_option["isOptional"]

        flagsJoined = ",".join(flags)

        cpp += f"""
        cliParams.{cpp_name} = {{}};
        app.add_option("{flagsJoined}", cliParams.{cpp_name}, "{desc}")->capture_default_str();
        """

        h += f"""
        {cpp_type} {cpp_name};
        """

    return (cpp, h)


def generate_comment(input_json):
    desc_json = os.path.relpath(input_json, PROJECT_ROOT_DIR)

    return \
        f"""
/*
 * !!! AUTOMATICALLY GENERATED CODE !!!
 * 
 * This file is autogenerated during build by
 *   {THIS_SCRIPT_REL}
 * using description in
 *   {desc_json}
 *
 * Do not edit this file. All manual edits will be overwritten!
 * Instead, edit {desc_json} and rebuild (which will run {THIS_SCRIPT_REL} and 
 * will generate this file)
 */
"""


def main():
    args = parse_args()

    with open(args.input_json, "r") as f:
        desc = json.load(f)

    comment = generate_comment(args.input_json)
    cpp, h = generate_code(desc)

    os.makedirs(os.path.dirname(args.output_cpp), exist_ok=True)
    with open(args.output_cpp, "w") as f:
        f.write(f"{comment}\n\n{CPP_HEADER}\n{cpp}\n{CPP_FOOTER}")

    os.makedirs(os.path.dirname(args.output_h), exist_ok=True)
    with open(args.output_h, "w") as f:
        f.write(f"{comment}\n\n{H_HEADER}\n{h}\n{H_FOOTER}")


if __name__ == '__main__':
    main()