// Spanda C++ FFI bridge (subprocess protocol v1).
// Reads JSON from stdin: {"fn":"cpp_add","args":[1,2]}
// Writes JSON to stdout: {"ok":true,"result":3}

#include <cmath>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

static std::string json_escape(const std::string &s) {
  std::string out;
  out.reserve(s.size() + 8);
  for (char c : s) {
    switch (c) {
    case '"':
      out += "\\\"";
      break;
    case '\\':
      out += "\\\\";
      break;
    default:
      out += c;
    }
  }
  return out;
}

static void respond_ok_number(double value) {
  std::cout << "{\"ok\":true,\"result\":" << value << "}\n";
}

static void respond_ok_string(const std::string &value) {
  std::cout << "{\"ok\":true,\"result\":\"" << json_escape(value) << "\"}\n";
}

static void respond_err(const std::string &msg) {
  std::cout << "{\"ok\":false,\"error\":\"" << json_escape(msg) << "\"}\n";
}

static std::string extract_fn_name(const std::string &json) {
  const std::string key = "\"fn\":\"";
  auto pos = json.find(key);
  if (pos == std::string::npos) {
    return {};
  }
  pos += key.size();
  auto end = json.find('"', pos);
  if (end == std::string::npos) {
    return {};
  }
  return json.substr(pos, end - pos);
}

static std::vector<double> extract_numeric_args(const std::string &json) {
  std::vector<double> args;
  const std::string key = "\"args\":[";
  auto start = json.find(key);
  if (start == std::string::npos) {
    return args;
  }
  start += key.size();
  auto end = json.find(']', start);
  if (end == std::string::npos) {
    return args;
  }
  std::string slice = json.substr(start, end - start);
  std::stringstream ss(slice);
  std::string token;
  while (std::getline(ss, token, ',')) {
    if (token.empty()) {
      continue;
    }
    try {
      args.push_back(std::stod(token));
    } catch (...) {
      // ignore non-numeric tokens in v1 bridge
    }
  }
  return args;
}

static bool dispatch(const std::string &fn, const std::vector<double> &args) {
  if (fn == "cpp_add") {
    if (args.size() < 2) {
      respond_err("cpp_add expects two numeric arguments");
      return true;
    }
    respond_ok_number(args[0] + args[1]);
    return true;
  }
  if (fn == "cpp_echo") {
    if (args.empty()) {
      respond_ok_number(0);
    } else {
      respond_ok_number(args[0]);
    }
    return true;
  }
  if (fn == "cpp_version") {
    respond_ok_number(1);
    return true;
  }
  return false;
}

int main() {
  std::string line;
  if (!std::getline(std::cin, line)) {
    respond_err("Missing JSON request on stdin");
    return 0;
  }

  const std::string fn = extract_fn_name(line);
  if (fn.empty()) {
    respond_err("Missing fn string in request");
    return 0;
  }

  const std::vector<double> args = extract_numeric_args(line);
  if (!dispatch(fn, args)) {
    respond_err("Unknown cpp extern '" + fn + "'");
  }
  return 0;
}
