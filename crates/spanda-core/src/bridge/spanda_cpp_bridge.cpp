// Spanda C++ FFI bridge (subprocess protocol v1 + optional in-process C API).
// Reads JSON from stdin: {"fn":"cpp_add","args":[1,2]}
// Writes JSON to stdout: {"ok":true,"result":3}

#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
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

static std::string ok_number_json(double value) {
  std::ostringstream oss;
  oss << "{\"ok\":true,\"result\":" << value << "}";
  return oss.str();
}

static std::string err_json(const std::string &msg) {
  return std::string("{\"ok\":false,\"error\":\"") + json_escape(msg) + "\"}";
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

static bool dispatch(const std::string &fn, const std::vector<double> &args,
                     std::string &out_json) {
  if (fn == "cpp_add") {
    if (args.size() < 2) {
      out_json = err_json("cpp_add expects two numeric arguments");
      return true;
    }
    out_json = ok_number_json(args[0] + args[1]);
    return true;
  }
  if (fn == "cpp_echo") {
    if (args.empty()) {
      out_json = ok_number_json(0);
    } else {
      out_json = ok_number_json(args[0]);
    }
    return true;
  }
  if (fn == "cpp_version") {
    out_json = ok_number_json(1);
    return true;
  }
  return false;
}

extern "C" int spanda_cpp_bridge_call(const char *fn_name, const char *args_json,
                                      char *out_buf, std::size_t out_len) {
  if (fn_name == nullptr || out_buf == nullptr || out_len == 0) {
    return 0;
  }
  std::string args_source =
      args_json != nullptr ? std::string(args_json) : std::string("{\"args\":[]}");
  const std::vector<double> args = extract_numeric_args(args_source);
  std::string response;
  if (!dispatch(fn_name, args, response)) {
    response = err_json(std::string("Unknown cpp extern '") + fn_name + "'");
  }
  if (response.size() + 1 > out_len) {
    return 0;
  }
  std::memcpy(out_buf, response.c_str(), response.size() + 1);
  return 1;
}

#ifndef SPANDA_CPP_LIBRARY
int main() {
  std::string line;
  if (!std::getline(std::cin, line)) {
    std::cout << err_json("Missing JSON request on stdin") << "\n";
    return 0;
  }

  const std::string fn = extract_fn_name(line);
  if (fn.empty()) {
    std::cout << err_json("Missing fn string in request") << "\n";
    return 0;
  }

  const std::vector<double> args = extract_numeric_args(line);
  std::string response;
  if (!dispatch(fn, args, response)) {
    response = err_json("Unknown cpp extern '" + fn + "'");
  }
  std::cout << response << "\n";
  return 0;
}
#endif
