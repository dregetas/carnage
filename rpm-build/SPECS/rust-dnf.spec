Name:           rust-dnf
Version:        0.1.0
Release:        1%{?dist}
Summary:        A DNF alternative written in Rust

License:        MIT OR Apache-2.0
URL:            https://github.com/yourusername/rust-dnf
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  systemd-rpm-macros
BuildRequires:  pkgconfig(openssl)

Requires:       systemd
Requires:       openssl

%description
A fast, modern package manager written in Rust, designed as an alternative to DNF.

%prep
%autosetup

%build
export CARGO_HOME=%{_builddir}/cargo-home
cargo build --release

%install
export CARGO_HOME=%{_builddir}/cargo-home

# Install the binary
install -D -m 0755 target/release/rust-dnf %{buildroot}%{_bindir}/rust-dnf

# Install configuration files
install -D -m 0644 rust-dnf.toml %{buildroot}%{_sysconfdir}/rust-dnf/config.toml

# Create necessary directories
mkdir -p %{buildroot}/var/cache/rust-dnf
mkdir -p %{buildroot}/var/lib/rust-dnf
mkdir -p %{buildroot}/etc/rust-dnf/repos.d

# Install RPM GPG keys (you'll need to provide these)
install -D -m 0644 RPM-GPG-KEY-rust-dnf %{buildroot}%{_sysconfdir}/pki/rpm-gpg/

%files
%license LICENSE-MIT LICENSE-APACHE
%doc README.md
%{_bindir}/rust-dnf
%{_sysconfdir}/rust-dnf/
%{_sysconfdir}/pki/rpm-gpg/RPM-GPG-KEY-rust-dnf
/var/cache/rust-dnf/
/var/lib/rust-dnf/

%config(noreplace) %{_sysconfdir}/rust-dnf/config.toml

%changelog
* Tue Dec 01 2023 Your Name <your.email@example.com> - 0.1.0-1
- Initial package build