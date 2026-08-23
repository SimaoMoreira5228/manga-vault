// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint, type=warning, deprecated_member_use, deprecated_member_use_from_same_package
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'local.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// GENERATED CODE - DO NOT MODIFY BY HAND
// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ChapterBody {

 Object get field0;



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ChapterBody&&const DeepCollectionEquality().equals(other.field0, field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(field0));

@override
String toString() {
  return 'ChapterBody(field0: $field0)';
}


}

/// @nodoc
class $ChapterBodyCopyWith<$Res>  {
$ChapterBodyCopyWith(ChapterBody _, $Res Function(ChapterBody) __);
}


/// Adds pattern-matching-related methods to [ChapterBody].
extension ChapterBodyPatterns on ChapterBody {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ChapterBody_Images value)?  images,TResult Function( ChapterBody_Html value)?  html,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ChapterBody_Images() when images != null:
return images(_that);case ChapterBody_Html() when html != null:
return html(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ChapterBody_Images value)  images,required TResult Function( ChapterBody_Html value)  html,}){
final _that = this;
switch (_that) {
case ChapterBody_Images():
return images(_that);case ChapterBody_Html():
return html(_that);}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ChapterBody_Images value)?  images,TResult? Function( ChapterBody_Html value)?  html,}){
final _that = this;
switch (_that) {
case ChapterBody_Images() when images != null:
return images(_that);case ChapterBody_Html() when html != null:
return html(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( List<String> field0)?  images,TResult Function( String field0)?  html,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ChapterBody_Images() when images != null:
return images(_that.field0);case ChapterBody_Html() when html != null:
return html(_that.field0);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( List<String> field0)  images,required TResult Function( String field0)  html,}) {final _that = this;
switch (_that) {
case ChapterBody_Images():
return images(_that.field0);case ChapterBody_Html():
return html(_that.field0);}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( List<String> field0)?  images,TResult? Function( String field0)?  html,}) {final _that = this;
switch (_that) {
case ChapterBody_Images() when images != null:
return images(_that.field0);case ChapterBody_Html() when html != null:
return html(_that.field0);case _:
  return null;

}
}

}

/// @nodoc


class ChapterBody_Images extends ChapterBody {
  const ChapterBody_Images( List<String> field0): _field0 = field0,super._();
  

 final  List<String> _field0;
@override List<String> get field0 {
  if (_field0 is EqualUnmodifiableListView) return _field0;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_field0);
}


/// Create a copy of ChapterBody
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ChapterBody_ImagesCopyWith<ChapterBody_Images> get copyWith => _$ChapterBody_ImagesCopyWithImpl<ChapterBody_Images>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ChapterBody_Images&&const DeepCollectionEquality().equals(other._field0, _field0));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_field0));

@override
String toString() {
  return 'ChapterBody.images(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $ChapterBody_ImagesCopyWith<$Res> implements $ChapterBodyCopyWith<$Res> {
  factory $ChapterBody_ImagesCopyWith(ChapterBody_Images value, $Res Function(ChapterBody_Images) _then) = _$ChapterBody_ImagesCopyWithImpl;
@useResult
$Res call({
 List<String> field0
});




}
/// @nodoc
class _$ChapterBody_ImagesCopyWithImpl<$Res>
    implements $ChapterBody_ImagesCopyWith<$Res> {
  _$ChapterBody_ImagesCopyWithImpl(this._self, this._then);

  final ChapterBody_Images _self;
  final $Res Function(ChapterBody_Images) _then;

/// Create a copy of ChapterBody
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(ChapterBody_Images(
null == field0 ? _self._field0 : field0 // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class ChapterBody_Html extends ChapterBody {
  const ChapterBody_Html(this.field0): super._();
  

@override final  String field0;

/// Create a copy of ChapterBody
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ChapterBody_HtmlCopyWith<ChapterBody_Html> get copyWith => _$ChapterBody_HtmlCopyWithImpl<ChapterBody_Html>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ChapterBody_Html&&(identical(other.field0, field0) || other.field0 == field0));
}


@override
int get hashCode => Object.hash(runtimeType,field0);

@override
String toString() {
  return 'ChapterBody.html(field0: $field0)';
}


}

/// @nodoc
abstract mixin class $ChapterBody_HtmlCopyWith<$Res> implements $ChapterBodyCopyWith<$Res> {
  factory $ChapterBody_HtmlCopyWith(ChapterBody_Html value, $Res Function(ChapterBody_Html) _then) = _$ChapterBody_HtmlCopyWithImpl;
@useResult
$Res call({
 String field0
});




}
/// @nodoc
class _$ChapterBody_HtmlCopyWithImpl<$Res>
    implements $ChapterBody_HtmlCopyWith<$Res> {
  _$ChapterBody_HtmlCopyWithImpl(this._self, this._then);

  final ChapterBody_Html _self;
  final $Res Function(ChapterBody_Html) _then;

/// Create a copy of ChapterBody
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? field0 = null,}) {
  return _then(ChapterBody_Html(
null == field0 ? _self.field0 : field0 // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on
